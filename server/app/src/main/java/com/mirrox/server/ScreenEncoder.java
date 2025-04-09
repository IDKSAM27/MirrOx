package com.mirrox.server;

import android.hardware.display.VirtualDisplay;
import android.media.MediaCodec;
import android.media.MediaCodecInfo;
import android.media.MediaFormat;
import android.media.projection.MediaProjection;
import android.util.Log;
import android.view.Surface;

import java.io.IOException;
import java.nio.ByteBuffer;

public class ScreenEncoder {
    private static final String TAG = "ScreenEncoder";
    private static final String MIME_TYPE = "video/avc"; // H.264
    private static final int FRAME_RATE = 30;
    private static final int IFRAME_INTERVAL = 1; // 1 second between I-frames
    private static final int TIMEOUT_US = 10000;

    private MediaProjection mediaProjection;
    private MediaCodec encoder;
    private Surface inputSurface;
    private VirtualDisplay virtualDisplay;
    private Thread encodingThread;
    private boolean isEncoding = false;

    public ScreenEncoder(MediaProjection mediaProjection) {
        this.mediaProjection = mediaProjection;
    }

    public void start(int width, int height) {
        try {
            MediaFormat format = MediaFormat.createVideoFormat(MIME_TYPE, width, height);
            format.setInteger(MediaFormat.KEY_COLOR_FORMAT,
                    MediaCodecInfo.CodecCapabilities.COLOR_FormatSurface);
            format.setInteger(MediaFormat.KEY_BIT_RATE, 4_000_000);
            format.setInteger(MediaFormat.KEY_FRAME_RATE, FRAME_RATE);
            format.setInteger(MediaFormat.KEY_I_FRAME_INTERVAL, IFRAME_INTERVAL);

            encoder = MediaCodec.createEncoderByType(MIME_TYPE);
            encoder.configure(format, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE);
            inputSurface = encoder.createInputSurface();
            encoder.start();

            virtualDisplay = mediaProjection.createVirtualDisplay(
                    "MirrOxDisplay",
                    width,
                    height,
                    320, // dpi
                    0,
                    inputSurface,
                    null,
                    null
            );

            isEncoding = true;
            encodingThread = new Thread(this::encodeLoop);
            encodingThread.start();

            Log.i(TAG, "Encoder started");
        } catch (IOException e) {
            Log.e(TAG, "Encoder initialization failed", e);
        }
    }

    private void encodeLoop() {
        MediaCodec.BufferInfo bufferInfo = new MediaCodec.BufferInfo();

        while (isEncoding) {
            int outputIndex = encoder.dequeueOutputBuffer(bufferInfo, TIMEOUT_US);
            if (outputIndex >= 0) {
                ByteBuffer encodedData = encoder.getOutputBuffer(outputIndex);
                if (encodedData != null && bufferInfo.size > 0) {
                    byte[] outData = new byte[bufferInfo.size];
                    encodedData.position(bufferInfo.offset);
                    encodedData.limit(bufferInfo.offset + bufferInfo.size);
                    encodedData.get(outData);

                    try {
                        System.out.write(outData);
                        System.out.flush();
                    } catch (IOException e) {
                        Log.e(TAG, "Failed to write to stdout", e);
                        break;
                    }
                }
                encoder.releaseOutputBuffer(outputIndex, false);
            }
        }
    }

    public void stop() {
        isEncoding = false;
        if (encodingThread != null) {
            try {
                encodingThread.join();
            } catch (InterruptedException e) {
                Log.e(TAG, "Encoding thread interrupted", e);
            }
        }

        if (virtualDisplay != null) virtualDisplay.release();
        if (encoder != null) encoder.stop();
        if (encoder != null) encoder.release();
        if (mediaProjection != null) mediaProjection.stop();

        Log.i(TAG, "Encoder stopped");
    }
}
