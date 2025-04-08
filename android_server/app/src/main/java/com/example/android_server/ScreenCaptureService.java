package com.example.android_server;

import android.app.Service;
import android.content.Intent;
import android.graphics.PixelFormat;
import android.hardware.display.DisplayManager;
import android.media.projection.MediaProjection;
import android.media.projection.MediaProjectionManager;
import android.os.IBinder;
import android.util.DisplayMetrics;
import android.view.Surface;
import android.view.WindowManager;
import android.media.ImageReader;
import android.media.Image;
import java.nio.ByteBuffer;
import java.io.ByteArrayOutputStream;
import android.graphics.Bitmap;
import android.app.Activity;
import android.content.Context;
import android.os.Build;

public class ScreenCaptureService extends Service {
    private MediaProjection mediaProjection;
    private ImageReader imageReader;
    private ScreenStreamingServer streamingServer;

    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        int resultCode = intent.getIntExtra("resultCode", Activity.RESULT_CANCELED);
        Intent data = intent.getParcelableExtra("data");

        MediaProjectionManager manager = null;
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            manager = (MediaProjectionManager) getSystemService(Context.MEDIA_PROJECTION_SERVICE);
        }

        if (manager == null) {
            stopSelf();
            return START_NOT_STICKY;
        }

        MediaProjection projection = manager.getMediaProjection(resultCode, data);
        if (projection == null) {
            stopSelf();
            return START_NOT_STICKY;
        }

        mediaProjection = projection;
        startScreenCapture();

        streamingServer = new ScreenStreamingServer();
        streamingServer.startServer();

        return START_STICKY;
    }


    private void startScreenCapture() {
        DisplayMetrics metrics = new DisplayMetrics();
        WindowManager windowManager = (WindowManager) getSystemService(WINDOW_SERVICE);
        if (windowManager != null) {
            windowManager.getDefaultDisplay().getMetrics(metrics);
        }

        int width = metrics.widthPixels;
        int height = metrics.heightPixels;

        imageReader = ImageReader.newInstance(width, height, PixelFormat.RGBA_8888, 2);
        Surface surface = imageReader.getSurface();
        mediaProjection.createVirtualDisplay("ScreenCapture",
                width, height, metrics.densityDpi,
                DisplayManager.VIRTUAL_DISPLAY_FLAG_AUTO_MIRROR,
                surface, null, null);

        imageReader.setOnImageAvailableListener(reader -> {
            Image image = reader.acquireLatestImage();
            if (image != null) {
                sendFrame(image);
                image.close();
            }
        }, null);
    }

    private void sendFrame(Image image) {
        Image.Plane[] planes = image.getPlanes();
        ByteBuffer buffer = planes[0].getBuffer();
        int width = image.getWidth();
        int height = image.getHeight();

        Bitmap bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888);
        bitmap.copyPixelsFromBuffer(buffer);

        ByteArrayOutputStream outputStream = new ByteArrayOutputStream();
        bitmap.compress(Bitmap.CompressFormat.JPEG, 50, outputStream);
        byte[] jpegData = outputStream.toByteArray();

        streamingServer.sendFrame(jpegData);
    }

    @Override
    public void onDestroy() {
        super.onDestroy();
        if (mediaProjection != null) {
            mediaProjection.stop();
        }
        if (streamingServer != null) {
            streamingServer.stopServer();
        }
    }
}

