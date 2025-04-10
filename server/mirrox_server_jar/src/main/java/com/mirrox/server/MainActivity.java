package com.mirrox.server;

import android.app.Activity;
import android.content.Intent;
import android.media.projection.MediaProjection;
import android.media.projection.MediaProjectionManager;
import android.os.Bundle;

public class MainActivity extends Activity {

    private static final int REQUEST_CODE = 1000;
    private MediaProjectionManager projectionManager;
    private ScreenEncoder encoder;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        projectionManager = (MediaProjectionManager) getSystemService(MEDIA_PROJECTION_SERVICE);
        Intent permissionIntent = projectionManager.createScreenCaptureIntent();
        startActivityForResult(permissionIntent, REQUEST_CODE);
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        if (requestCode == REQUEST_CODE && resultCode == RESULT_OK) {
            MediaProjection mediaProjection = projectionManager.getMediaProjection(resultCode, data);
            encoder = new ScreenEncoder(mediaProjection);
            encoder.start(1080, 1920); // change to your actual screen resolution
        } else {
            System.err.println("Screen capture permission denied.");
            finish();
        }
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        if (encoder != null) {
            encoder.stop();
        }
    }
}
