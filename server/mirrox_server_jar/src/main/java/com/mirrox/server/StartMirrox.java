package com.mirrox.server;

import android.content.Context;
import android.media.projection.MediaProjection;
import android.media.projection.MediaProjectionManager;
import android.os.IBinder;
import android.os.IInterface;
import android.os.Looper;
import android.os.Parcel;

import java.lang.reflect.Method;

public class StartMirrox {

    private static final int SCREEN_WIDTH = 720;
    private static final int SCREEN_HEIGHT = 1280;

    public static void main(String[] args) {
        System.out.println("✅ MirrOx Server Started using main()");
        try {
            Context context = getSystemContext();
            MediaProjection mediaProjection = getMediaProjectionFromBinder(context);
            if (mediaProjection == null) {
                System.err.println("❌ Failed to get MediaProjection");
                return;
            }

            ScreenEncoder encoder = new ScreenEncoder(mediaProjection);
            encoder.start(SCREEN_WIDTH, SCREEN_HEIGHT);

            // Keep the server alive
            while (true) {
                Thread.sleep(1000);
            }

        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    // Get system context like scrcpy
    private static Context getSystemContext() throws Exception {
        Looper.prepareMainLooper();

        Class<?> activityThreadClass = Class.forName("android.app.ActivityThread");
        Method systemMain = activityThreadClass.getMethod("systemMain");
        Object activityThread = systemMain.invoke(null);

        Method getSystemContext = activityThreadClass.getMethod("getSystemContext");
        return (Context) getSystemContext.invoke(activityThread);
    }

    // Scrcpy-style raw Binder IPC to request MediaProjection
    private static MediaProjection getMediaProjectionFromBinder(Context context) {
        try {
            IBinder binder = (IBinder) Class.forName("android.os.ServiceManager")
                    .getMethod("getService", String.class)
                    .invoke(null, Context.MEDIA_PROJECTION_SERVICE);

            if (binder == null) {
                System.err.println("❌ Could not get media_projection service");
                return null;
            }

            Parcel data = Parcel.obtain();
            Parcel reply = Parcel.obtain();

            // Must match the AIDL interface token
            data.writeInterfaceToken("android.media.projection.IMediaProjectionManager");
            data.writeInt(2000); // shell UID
            data.writeString("com.mirrox.server"); // your shell-granted package name

            binder.transact(1, data, reply, 0);

            reply.readException();
            IBinder projectionBinder = reply.readStrongBinder();

            // Wrap the returned binder in the MediaProjection system service
            MediaProjectionManager mpm = (MediaProjectionManager) context.getSystemService(Context.MEDIA_PROJECTION_SERVICE);
            Method asInterface = Class.forName("android.media.projection.IMediaProjection$Stub")
                    .getMethod("asInterface", IBinder.class);
            IInterface projection = (IInterface) asInterface.invoke(null, projectionBinder);

            Method getMediaProjection = mpm.getClass().getDeclaredMethod("getMediaProjection", int.class, IInterface.class);
            getMediaProjection.setAccessible(true);

            return (MediaProjection) getMediaProjection.invoke(mpm, 0, projection);

        } catch (Exception e) {
            e.printStackTrace();
            return null;
        }
    }
}
