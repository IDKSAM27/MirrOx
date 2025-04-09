package com.mirrox.server;

import android.annotation.SuppressLint;
import android.content.Context;
import android.hardware.display.DisplayManager;
import android.hardware.display.VirtualDisplay;
import android.media.MediaCodec;
import android.media.MediaCodecInfo;
import android.media.MediaFormat;
import android.media.projection.MediaProjection;
import android.util.DisplayMetrics;
import android.util.Log;
import android.view.Surface;

import java.io.IOException;
import java.nio.ByteBuffer;

public class ScreenEncoder {
    private static final String TAG = "ScreenEncoder";

    private static final String VIDEO_MIME_TYPE = "video/avc"; // H.264
    private static final int BIT_RATE = 4_000_000; // 4Mbps
    private static final int FRAME_RATE = 30;
    private static final int I_FRAME_INTERVAL = 1; // seconds

    private MediaProjection mediaProjection;
    private VirtualDisplay virtualDisplay;
    private MediaCodec videoEncoder;
    private Surface inputSurface;

    public ScreenEncoder(MediaProjection mediaProjection) {
        this.mediaProjection = mediaProjection;
    }

    public void start(int width, int height) {
        try {
            // 1. Configure the MediaCodec encoder
            MediaFormat format = MediaFormat.createVideoFormat(VIDEO_MIME_TYPE, width, height);
            format.setInteger(MediaFormat.KEY_COLOR_FORMAT,
                    MediaCodecInfo.CodecCapabilities.COLOR_FormatSurface);
            format.setInteger(MediaFormat.KEY_BIT_RATE, BIT_RATE);
            format.setInteger(MediaFormat.KEY_FRAME_RATE, FRAME_RATE);
            format.setInteger(MediaFormat.KEY_I_FRAME_INTERVAL, I_FRAME_INTERVAL);

            videoEncoder = MediaCodec.createEncoderByType(VIDEO_MIME_TYPE);
            videoEncoder.configure(format, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE);
            inputSurface = videoEncoder.createInputSurface();
            videoEncoder.start();

            // 2. Create VirtualDisplay to capture the screen into the encoder's input surface
            virtualDisplay = mediaProjection.createVirtualDisplay(
                    "MirrOxDisplay",
                    width, height, getDensityDpi(),
                    DisplayManager.VIRTUAL_DISPLAY_FLAG_AUTO_MIRROR,
                    inputSurface,
                    null, null
            );

            // 3. Start a new thread to process the encoded output
            new Thread(this::encodeLoop).start();

        } catch (IOException e) {
            Log.e(TAG, "Failed to start encoder", e);
        }
    }

    private void encodeLoop() {
        MediaCodec.BufferInfo bufferInfo = new MediaCodec.BufferInfo();

        while (true) {
            int outputBufferId = videoEncoder.dequeueOutputBuffer(bufferInfo, 10_000);
            if (outputBufferId >= 0) {
                ByteBuffer encodedData = videoEncoder.getOutputBuffer(outputBufferId);
                if (encodedData != null) {
                    byte[] buffer = new byte[bufferInfo.size];
                    encodedData.get(buffer);

                    // TODO: Stream this buffer to stdout or a socket
                    // Example: System.out.write(buffer);

                    videoEncoder.releaseOutputBuffer(outputBufferId, false);
                }
            }
        }
    }

    @SuppressLint("WrongConstant") // Suppresses the Dpi constant error
    private int getDensityDpi() {
        DisplayMetrics metrics = new DisplayMetrics();
        // Dummy context for DPI; will override from Activity
        metrics.densityDpi = 320;
        return metrics.densityDpi;
    }

    public void stop() {
        if (virtualDisplay != null) virtualDisplay.release();
        if (videoEncoder != null) videoEncoder.stop();
        if (mediaProjection != null) mediaProjection.stop();
    }
}
