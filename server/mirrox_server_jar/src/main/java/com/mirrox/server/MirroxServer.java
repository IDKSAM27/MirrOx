package com.mirrox.server;

public class MirroxServer {
    public static void main(String[] args) {
        System.out.println("MirrOx server started");

        // TODO: Hook into MediaProjection, start encoder, etc.

        // For now, just wait to prevent app from exiting
        try {
            Thread.sleep(10000); // keep alive for 10s
        } catch (InterruptedException e) {
            e.printStackTrace();
        }

        System.out.println("MirrOx server exiting");
    }
}
