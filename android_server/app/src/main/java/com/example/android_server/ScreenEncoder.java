package com.example.android_server;

import android.media.MediaCodec;
import android.media.MediaCodecInfo;
import android.media.MediaFormat;
import android.util.Log;
import java.io.IOException;
import java.nio.ByteBuffer;

public class ScreenEncoder {
    private static final String TAG = "ScreenEncoder";
    private static final String MIME_TYPE = "video/avc"; // H.264
    private static final int WIDTH = 1280;
    private static final int HEIGHT = 720;
    private static final int FRAME_RATE = 30;
    private static final int BIT_RATE = 5_000_000;
    private static final int I_FRAME_INTERVAL = 2;

    private MediaCodec mediaCodec;

    public ScreenEncoder() {
        try {
            MediaFormat format = MediaFormat.createVideoFormat(MIME_TYPE, WIDTH, HEIGHT);
            format.setInteger(MediaFormat.KEY_COLOR_FORMAT, MediaCodecInfo.CodecCapabilities.COLOR_FormatSurface);
            format.setInteger(MediaFormat.KEY_BIT_RATE, BIT_RATE);
            format.setInteger(MediaFormat.KEY_FRAME_RATE, FRAME_RATE);
            format.setInteger(MediaFormat.KEY_I_FRAME_INTERVAL, I_FRAME_INTERVAL);

            mediaCodec = MediaCodec.createEncoderByType(MIME_TYPE);
            mediaCodec.configure(format, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE);
        } catch (IOException e) {
            Log.e(TAG, "Failed to initialize MediaCodec", e);
        }
    }

    public MediaCodec getEncoder() {
        return mediaCodec;
    }

    public void startEncoding() {
        if (mediaCodec != null) {
            mediaCodec.start();
        }
    }

    public ByteBuffer getEncodedData() {
        MediaCodec.BufferInfo bufferInfo = new MediaCodec.BufferInfo();
        int outputIndex = mediaCodec.dequeueOutputBuffer(bufferInfo, 0);
        if (outputIndex >= 0) {
            ByteBuffer encodedData = mediaCodec.getOutputBuffer(outputIndex);
            mediaCodec.releaseOutputBuffer(outputIndex, false);
            return encodedData;
        }
        return null;
    }

    public void stopEncoding() {
        if (mediaCodec != null) {
            mediaCodec.stop();
            mediaCodec.release();
        }
    }
}
