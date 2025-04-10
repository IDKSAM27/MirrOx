package com.mirrox.server;

import android.app.Service;
import android.content.Intent;
import android.media.projection.MediaProjection;
import android.media.projection.MediaProjectionManager;
import android.os.IBinder;
import android.util.Log;

public class ScreenCaptureService extends Service {

    private static final String TAG = "MirrOxServer";
    private MediaProjection mediaProjection;

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        int resultCode = intent.getIntExtra("resultCode", 0);
        Intent data = intent.getParcelableExtra("data");

        MediaProjectionManager mpm = (MediaProjectionManager) getSystemService(MEDIA_PROJECTION_SERVICE);
        mediaProjection = mpm.getMediaProjection(resultCode, data);

        if (mediaProjection == null) {
            Log.e(TAG, "MediaProjection is null. Exiting.");
            stopSelf();
            return START_NOT_STICKY;
        }

        Log.i(TAG, "MediaProjection acquired, starting encoder...");

        // TODO: Start encoder thread
        // new ScreenEncoder(mediaProjection).start();

        return START_STICKY;
    }

    @Override
    public IBinder onBind(Intent intent) {
        return null; // We don’t support binding
    }

    @Override
    public void onDestroy() {
        super.onDestroy();
        if (mediaProjection != null) {
            mediaProjection.stop();
        }
    }
}
