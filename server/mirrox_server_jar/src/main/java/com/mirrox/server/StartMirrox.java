package com.mirrox.server;

public class StartMirrox {

    static {
        System.load("/data/local/tmp/librust_binder.so");
    }

    public static native void getMediaProjectionTokenNative();

    public static void main(String[] args) {
        System.out.println("✅ MirrOx Server Started using main()");

        // Call native method that triggers raw Binder IPC
        getMediaProjectionTokenNative();

        System.out.println("📣 Native call finished.");
    }
}
