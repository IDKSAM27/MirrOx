package com.example.android_server;

import java.io.OutputStream;
import java.net.ServerSocket;
import java.net.Socket;
import android.app.Service;
import android.content.Intent;
import android.graphics.PixelFormat;
import android.hardware.display.DisplayManager;
import android.media.projection.MediaProjection;
import android.media.projection.MediaProjectionManager;
import android.os.IBinder;
import android.view.Surface;
import android.view.SurfaceView;
import android.view.WindowManager;

public class ScreenCaptureService extends Service {
    private static final int PORT = 8080;
    private ServerSocket serverSocket;
    private Socket clientSocket;
    private OutputStream outputStream;
    private MediaProjection mediaProjection;
    private SurfaceView surfaceView;

    @Override
    public void onCreate() {
        super.onCreate();
        startServer();
    }

    private void startServer() {
        new Thread(() -> {
            try {
                serverSocket = new ServerSocket(PORT);
                clientSocket = serverSocket.accept();
                outputStream = clientSocket.getOutputStream();
            } catch (Exception e) {
                e.printStackTrace();
            }
        }).start();
    }

    private void sendFrame(byte[] data) {
        try {
            if (outputStream != null) {
                outputStream.write(data);
                outputStream.flush();
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        MediaProjectionManager projectionManager = (MediaProjectionManager) getSystemService(MEDIA_PROJECTION_SERVICE);
        int resultCode = intent.getIntExtra("RESULT_CODE", -1);
        Intent data = intent.getParcelableExtra("DATA");

        if (projectionManager != null && resultCode != -1 && data != null) {
            mediaProjection = projectionManager.getMediaProjection(resultCode, data);
            setupVirtualDisplay();
        }
        return START_STICKY;
    }

    private void setupVirtualDisplay() {
        WindowManager windowManager = (WindowManager) getSystemService(WINDOW_SERVICE);
        surfaceView = new SurfaceView(this);
        Surface surface = surfaceView.getHolder().getSurface();

        mediaProjection.createVirtualDisplay("ScreenCapture",
                1080, 1920, 1,
                DisplayManager.VIRTUAL_DISPLAY_FLAG_AUTO_MIRROR,
                surface, null, null);
    }

    @Override
    public void onDestroy() {
        super.onDestroy();
        try {
            if (clientSocket != null) clientSocket.close();
            if (serverSocket != null) serverSocket.close();
        } catch (Exception e) {
            e.printStackTrace();
        }
        if (mediaProjection != null) {
            mediaProjection.stop();
        }
    }

    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }
}
