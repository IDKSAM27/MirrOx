package com.mirrox.server;

import android.content.Context;
// import android.hardware.display.DisplayManager;
import android.media.projection.MediaProjection;
import android.media.projection.MediaProjectionManager;
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
        // Prepare the main looper before systemMain to avoid Handler crash
        Looper.prepareMainLooper();

        Class<?> activityThreadClass = Class.forName("android.app.ActivityThread");
        Method method = activityThreadClass.getMethod("systemMain");
        Object activityThread = method.invoke(null);

        Method getSystemContext = activityThreadClass.getMethod("getSystemContext");
        return (Context) getSystemContext.invoke(activityThread);
    }


        // Get MediaProjection without UI — requires shell-granted permission
        private static MediaProjection getMediaProjection(Context context) throws Exception {
        // 1. Get the IMediaProjectionManager binder
        IBinder projectionService = (IBinder) Class.forName("android.os.ServiceManager")
                .getMethod("getService", String.class)
                .invoke(null, "media_projection");

        if (projectionService == null) {
            System.err.println("❌ media_projection service not available");
            return null;
        }

        // 2. Convert binder to IMediaProjectionManager
        Class<?> stubClass = Class.forName("android.media.projection.IMediaProjectionManager$Stub");
        Method asInterface = stubClass.getMethod("asInterface", IBinder.class);
        Object iMediaProjectionManager = asInterface.invoke(null, projectionService);

        // 3. Call createProjection(int uid, String packageName, int type, boolean permanentGrant)
        Method createProjection = iMediaProjectionManager.getClass().getMethod(
                "createProjection", int.class, String.class, int.class, boolean.class);

        // This mimics scrcpy: uid=2000 (shell), packageName="com.mirrox.shell", type=0 (screen capture), permanentGrant=true
        Object projection = createProjection.invoke(iMediaProjectionManager, 2000, "com.mirrox.shell", 0, true);

        // 4. Get MediaProjectionManager and wrap token
        MediaProjectionManager mgr = (MediaProjectionManager) context.getSystemService(Context.MEDIA_PROJECTION_SERVICE);
        Method getMediaProjection = mgr.getClass().getDeclaredMethod("getMediaProjection", int.class, IBinder.class);
        getMediaProjection.setAccessible(true);

        return (MediaProjection) getMediaProjection.invoke(mgr, 0, (IBinder) projection);
    }

    
    
    
}
