package com.mirrox.server;

import android.app.Activity;
import android.content.Context;
import android.content.Intent;
import android.media.projection.MediaProjection;
import android.media.projection.MediaProjectionManager;
import android.os.Bundle;
import android.util.DisplayMetrics;
import android.util.Log;
import android.view.WindowManager;

public class MainActivity extends Activity {
    private static final int REQUEST_CODE_SCREEN_CAPTURE = 1000;
    private static final String TAG = "MainActivity";

    private MediaProjectionManager projectionManager;
    private ScreenEncoder screenEncoder;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        // Initialize MediaProjectionManager
        projectionManager = (MediaProjectionManager) getSystemService(Context.MEDIA_PROJECTION_SERVICE);

        // Start screen capture intent
        Intent captureIntent = projectionManager.createScreenCaptureIntent();
        startActivityForResult(captureIntent, REQUEST_CODE_SCREEN_CAPTURE);
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        if (requestCode == REQUEST_CODE_SCREEN_CAPTURE) {
            if (resultCode == RESULT_OK && data != null) {
                MediaProjection mediaProjection = projectionManager.getMediaProjection(resultCode, data);

                // Get screen dimensions
                DisplayMetrics metrics = new DisplayMetrics();
                WindowManager windowManager = (WindowManager) getSystemService(Context.WINDOW_SERVICE);
                windowManager.getDefaultDisplay().getMetrics(metrics);
                int screenWidth = metrics.widthPixels;
                int screenHeight = metrics.heightPixels;

                // Start the encoder
                screenEncoder = new ScreenEncoder(mediaProjection);
                screenEncoder.start(screenWidth, screenHeight);

                Log.i(TAG, "Screen capture started");
            } else {
                Log.e(TAG, "User denied screen capture permission");
                finish(); // Exit if permission is not granted
            }
        }
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        if (screenEncoder != null) {
            screenEncoder.stop();
        }
    }
}
