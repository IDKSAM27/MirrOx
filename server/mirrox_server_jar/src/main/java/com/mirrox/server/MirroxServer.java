package com.mirrox.server;

import android.util.Log;

public class MirroxServer {
    public static void main(String[] args) {
        try {
            Log.i("MirroxServer", "Hello from Mirrox Server!");
            System.out.println("MirrOx server started");

            // Simulate long-running server
            Thread.sleep(10000);

            System.out.println("MirrOx server exiting");
        } catch (Exception e) {
            System.err.println("MirrOx Server Error: " + e.getMessage());
            e.printStackTrace();
        }
    }
}
