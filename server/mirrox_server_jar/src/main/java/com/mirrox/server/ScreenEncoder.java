package com.mirrox.server;

import android.hardware.display.DisplayManager;
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

    private static final String TAG = "MirroxScreenEncoder";

    private static final String MIME_TYPE = "video/avc"; // H.264
    private static final int FRAME_RATE = 30;
    private static final int IFRAME_INTERVAL = 1;
    private static final int BIT_RATE = 5 * 1024 * 1024; // 5 Mbps

    private final MediaProjection mediaProjection;
    private MediaCodec mediaCodec;
    private VirtualDisplay virtualDisplay;

    public ScreenEncoder(MediaProjection projection) {
        this.mediaProjection = projection;
    }

    public void start(int width, int height) {
        try {
            MediaFormat format = MediaFormat.createVideoFormat(MIME_TYPE, width, height);
            format.setInteger(MediaFormat.KEY_COLOR_FORMAT,
                    MediaCodecInfo.CodecCapabilities.COLOR_FormatSurface);
            format.setInteger(MediaFormat.KEY_BIT_RATE, BIT_RATE);
            format.setInteger(MediaFormat.KEY_FRAME_RATE, FRAME_RATE);
            format.setInteger(MediaFormat.KEY_I_FRAME_INTERVAL, IFRAME_INTERVAL);

            mediaCodec = MediaCodec.createEncoderByType(MIME_TYPE);
            mediaCodec.configure(format, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE);
            Surface inputSurface = mediaCodec.createInputSurface();
            mediaCodec.start();

            virtualDisplay = mediaProjection.createVirtualDisplay(
                    "MirrOxDisplay",
                    width, height, 320,
                    DisplayManager.VIRTUAL_DISPLAY_FLAG_PUBLIC,
                    inputSurface,
                    null, null
            );

            new Thread(this::encodeLoop).start();

        } catch (IOException e) {
            Log.e(TAG, "Error setting up screen encoder", e);
        }
    }

    private void encodeLoop() {
        MediaCodec.BufferInfo bufferInfo = new MediaCodec.BufferInfo();

        while (true) {
            int outputBufferId = mediaCodec.dequeueOutputBuffer(bufferInfo, 10000);
            if (outputBufferId >= 0) {
                ByteBuffer encodedData = mediaCodec.getOutputBuffer(outputBufferId);
                if (encodedData != null && bufferInfo.size > 0) {
                    encodedData.position(bufferInfo.offset);
                    encodedData.limit(bufferInfo.offset + bufferInfo.size);

                    // You can stream/write the encodedData here
                    Log.d(TAG, "Encoded frame size: " + bufferInfo.size);
                }
                mediaCodec.releaseOutputBuffer(outputBufferId, false);
            }
        }
    }

    public void stop() {
        if (virtualDisplay != null) {
            virtualDisplay.release();
        }
        if (mediaCodec != null) {
            mediaCodec.stop();
            mediaCodec.release();
        }
        if (mediaProjection != null) {
            mediaProjection.stop();
        }
    }
}
