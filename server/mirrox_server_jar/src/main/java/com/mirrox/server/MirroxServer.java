package com.mirrox.server;

import android.app.Application;
import android.os.Handler;

public class MirroxServer extends Application {

    @Override
    public void onCreate() {
        super.onCreate();

        System.out.println("✅ MirrOx Server Started inside Android Application");

        // Run your server logic here (on another thread if needed)
        new Handler().post(() -> {
            try {
                // Your screen encoder / MediaProjection logic here
                System.out.println("👀 Running MirrOx Server logic...");
            } catch (Exception e) {
                e.printStackTrace();
            }
        });
    }
}
