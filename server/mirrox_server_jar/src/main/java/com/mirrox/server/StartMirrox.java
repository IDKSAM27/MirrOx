package com.mirrox.server;

import android.content.Context;
// import android.hardware.display.DisplayManager;
import android.media.projection.MediaProjection;
import android.os.IBinder;
// import android.os.ServiceManager;
import android.os.Looper;

import java.lang.reflect.Method;

public class StartMirrox {

    private static final int SCREEN_WIDTH = 720;
    private static final int SCREEN_HEIGHT = 1280;

    public static void main(String[] args) {
        System.out.println("✅ MirrOx Server Started using main()");
        try {
            Context context = getSystemContext();
            MediaProjection mediaProjection = getMediaProjection();
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
    // Prepare the main looper before systemMain to avoid Handler crash
    Looper.prepareMainLooper();

    Class<?> activityThreadClass = Class.forName("android.app.ActivityThread");
    Method method = activityThreadClass.getMethod("systemMain");
    Object activityThread = method.invoke(null);

    Method getSystemContext = activityThreadClass.getMethod("getSystemContext");
    return (Context) getSystemContext.invoke(activityThread);
}


    // Get MediaProjection without UI — requires shell-granted permission
    private static MediaProjection getMediaProjection() {
        try {
            // Get the media_projection service binder
            IBinder projectionService = (IBinder) Class.forName("android.os.ServiceManager")
                    .getMethod("getService", String.class)
                    .invoke(null, "media_projection");
    
            if (projectionService == null) {
                System.err.println("❌ media_projection service not available");
                return null;
            }
    
            // Get the stub class: IMediaProjectionManager.Stub
            Class<?> stubClass = Class.forName("android.media.projection.IMediaProjectionManager$Stub");
            Object iMediaProjectionManager = stubClass
                    .getMethod("asInterface", IBinder.class)
                    .invoke(null, projectionService);
    
            // Create a fake package name and UID (shell UID = 2000)
            String packageName = "com.mirrox.shell";
            int uid = 2000; // shell UID
            int displayId = 0; // virtual display
    
            // Call createProjection(int uid, String packageName, int type, boolean permanentGrant)
            Method createProjection = iMediaProjectionManager.getClass()
                    .getMethod("createProjection", int.class, String.class, int.class, boolean.class);
    
            Object iMediaProjection = createProjection.invoke(iMediaProjectionManager, uid, packageName, 0, true);
    
            // Now wrap it with MediaProjection
            Class<?> mediaProjectionClass = Class.forName("android.media.projection.MediaProjection");
            Method getInstance = mediaProjectionClass.getDeclaredMethod("getInstance", int.class, Object.class);
            getInstance.setAccessible(true);
    
            return (MediaProjection) getInstance.invoke(null, 0, iMediaProjection);
    
        } catch (Exception e) {
            e.printStackTrace();
            return null;
        }
    }
    
    
    
}
