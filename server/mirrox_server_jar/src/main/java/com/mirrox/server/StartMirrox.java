package com.mirrox.server;

import android.content.Context;
// import android.hardware.display.DisplayManager;
import android.media.projection.MediaProjection;
import android.media.projection.MediaProjectionManager;
import android.os.IBinder;
// import android.os.ServiceManager;

import java.lang.reflect.Method;

public class StartMirrox {

    private static final int SCREEN_WIDTH = 720;
    private static final int SCREEN_HEIGHT = 1280;

    public static void main(String[] args) {
        System.out.println("✅ MirrOx Server Started using main()");
        try {
            Context context = getSystemContext();
            MediaProjection mediaProjection = getMediaProjection(context);
            if (mediaProjection == null) {
                System.err.println("❌ Failed to get MediaProjection");
                return;
            }

            ScreenEncoder encoder = new ScreenEncoder(mediaProjection);
            encoder.start(SCREEN_WIDTH, SCREEN_HEIGHT);

            // Keep the process alive (simulate long-running server)
            while (true) {
                Thread.sleep(1000);
            }

        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    // Get system context like scrcpy
    private static Context getSystemContext() throws Exception {
        Class<?> activityThread = Class.forName("android.app.ActivityThread");
        Method systemMain = activityThread.getMethod("systemMain");
        Object activityThreadInstance = systemMain.invoke(null);
        Method getSystemContext = activityThread.getMethod("getSystemContext");
        return (Context) getSystemContext.invoke(activityThreadInstance);
    }

    // Get MediaProjection without UI — requires shell-granted permission
    private static MediaProjection getMediaProjection(Context context) {
        MediaProjectionManager projectionManager =
                (MediaProjectionManager) context.getSystemService(Context.MEDIA_PROJECTION_SERVICE);
        try {
            // Reflectively call android.os.ServiceManager.getService("media_projection")
            IBinder projectionToken = (IBinder) Class.forName("android.os.ServiceManager")      // ServiceManager is actually is part of the internal ANDROID SDK and not accessible via android.jar.
                    .getMethod("getService", String.class)                         // Instead of importing android.os.ServiceManager directly, we can use reflection to access it
                    .invoke(null, "media_projection");
    
            if (projectionToken != null) {
                Method getMediaProjection =
                        projectionManager.getClass().getDeclaredMethod("getMediaProjection", int.class, IBinder.class);
                getMediaProjection.setAccessible(true);
                return (MediaProjection) getMediaProjection.invoke(projectionManager, 0, projectionToken);
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
        return null;
    }
    
}
