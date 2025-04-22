package com.mirrox.server;

import android.os.IBinder;

public class StartMirrox {

    // Load the native .so library
    static {
        System.load("/data/local/tmp/mirrox_libs/libmirroxjni.so");
    }

    // Declare the native method
    public static native IBinder getMediaProjectionTokenNative();

    public static void main(String[] args) {
        System.out.println("✅ MirrOx Server Started using main()");

        // Call the native method and print its result
        IBinder projectionToken = getMediaProjectionTokenNative();
        System.out.println("📣 JNI result: " + projectionToken);
    }
}
