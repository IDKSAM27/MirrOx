package com.example.android_server;

import android.app.Service;
import android.content.Intent;
import android.os.IBinder;
import android.util.Log;

public class ScreenCaptureService extends Service {
    private static final String TAG = "ScreenCaptureService";
    private ScreenEncoder screenEncoder;

    @Override
    public void onCreate() {
        super.onCreate();
        Log.d(TAG, "ScreenCaptureService started");
        screenEncoder = new ScreenEncoder();
        screenEncoder.startEncoding();
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        return START_STICKY;
    }

    @Override
    public void onDestroy() {
        super.onDestroy();
        if (screenEncoder != null) {
            screenEncoder.stopEncoding();
        }
        Log.d(TAG, "ScreenCaptureService stopped");
    }

    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }
}
