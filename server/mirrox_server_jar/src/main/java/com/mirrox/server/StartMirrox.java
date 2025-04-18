package com.mirrox.server;

public class StartMirrox {

    // Load the native .so library
    static {
        System.load("/data/local/tmp/mirrox_libs/libmirroxjni.so");
    }

    // Declare the native method
    public static native int startMediaProjection();

    public static void main(String[] args) {
        System.out.println("✅ MirrOx Server Started using main()");

        // Call the native method and print its result
        int result = startMediaProjection();
        System.out.println("📣 JNI result: " + result);
    }
}
