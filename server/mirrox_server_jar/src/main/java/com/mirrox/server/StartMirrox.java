package com.mirrox.server;

import android.os.IBinder;

public class StartMirrox {

    static {
        System.load("/data/local/tmp/librust_binder.so");
    }

    public static native IBinder getMediaProjectionTokenNative();

    public static void main(String[] args) {
        System.out.println("✅ MirrOx Server Started using main()");
        IBinder token = getMediaProjectionTokenNative();
        System.out.println("📣 JNI result: " + token);
    }
}
