package com.mirrox.server;

import android.os.IBinder;

public class StartMirrox {
    static {
        System.load("/data/local/tmp/librust_binder.so");
    }

    public static void main(String[] args) {
        System.out.println("✅ MirrOx Server Started using main()");

        System.out.println("📡 Native getMediaProjectionTokenNative called");
        IBinder token = getMediaProjectionTokenNative();
        System.out.println("📣 JNI result: " + token);

        // TODO: Use this token in your encoder once it's ready
    }

    public static native IBinder getMediaProjectionTokenNative();
}
