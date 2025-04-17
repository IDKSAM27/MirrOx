package com.mirrox.server;

import android.content.Context;
import android.media.projection.MediaProjection;
import android.media.projection.MediaProjectionManager;
import android.os.IBinder;
import android.os.Looper;
import java.lang.reflect.Method;

public class StartMirrox {

    static {
        System.loadLibrary("mirroxjni"); // libmirroxjni.so
    }

    public static native IBinder getMediaProjectionTokenNative();

    public static void main(String[] args) {
        System.out.println("✅ MirrOx Server Started using main()");

        try {
            Context context = getSystemContext();

            IBinder projectionToken = getMediaProjectionTokenNative();
            if (projectionToken == null) {
                System.err.println("❌ Failed to obtain projection token");
                return;
            }

            MediaProjectionManager mpm =
                    (MediaProjectionManager) context.getSystemService(Context.MEDIA_PROJECTION_SERVICE);

            Method getProjection = MediaProjectionManager.class.getDeclaredMethod(
                    "getMediaProjection", int.class, IBinder.class);
            getProjection.setAccessible(true);
            MediaProjection mp = (MediaProjection) getProjection.invoke(mpm, 0, projectionToken);

            // start encoder
            ScreenEncoder encoder = new ScreenEncoder(mp);
            encoder.start(720, 1280);

            while (true) {
                Thread.sleep(1000);
            }

        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    private static Context getSystemContext() throws Exception {
        Looper.prepareMainLooper();
        Class<?> activityThread = Class.forName("android.app.ActivityThread");
        Method systemMain = activityThread.getMethod("systemMain");
        Object at = systemMain.invoke(null);
        Method getContext = activityThread.getMethod("getSystemContext");
        return (Context) getContext.invoke(at);
    }
}

