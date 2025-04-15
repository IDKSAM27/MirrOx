package com.mirrox.server;

import android.content.Context;
import android.media.projection.MediaProjection;
import android.os.IBinder;
import android.os.IInterface;
import android.os.Looper;
import android.os.Parcel;

import java.lang.reflect.Constructor;
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

            while (true) {
                Thread.sleep(1000);
            }

        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    private static Context getSystemContext() throws Exception {
        Looper.prepareMainLooper();

        Class<?> activityThreadClass = Class.forName("android.app.ActivityThread");
        Method systemMain = activityThreadClass.getMethod("systemMain");
        Object activityThread = systemMain.invoke(null);

        Method getSystemContext = activityThreadClass.getMethod("getSystemContext");
        return (Context) getSystemContext.invoke(activityThread);
    }

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

            data.writeInterfaceToken("android.media.projection.IMediaProjectionManager");
            data.writeInt(2000); // UID for shell
            data.writeString("com.mirrox.server");

            binder.transact(1, data, reply, 0);
            reply.readException();
            IBinder projectionBinder = reply.readStrongBinder();

            // Bind to IMediaProjection
            Class<?> stubClass = Class.forName("android.media.projection.IMediaProjection$Stub");
            Method asInterface = stubClass.getMethod("asInterface", IBinder.class);
            IInterface iMediaProjection = (IInterface) asInterface.invoke(null, projectionBinder);

            // Use hidden constructor of MediaProjection
            Constructor<MediaProjection> constructor = MediaProjection.class.getDeclaredConstructor(int.class, IInterface.class);
            constructor.setAccessible(true);
            return constructor.newInstance(0, iMediaProjection);

        } catch (Exception e) {
            e.printStackTrace();
            return null;
        }
    }
}
