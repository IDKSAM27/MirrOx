package com.mirrox.server;

import android.media.projection.IMediaProjection;
import android.media.projection.IMediaProjectionManager;
import android.os.IBinder;
import android.os.ServiceManager;

public class StartMirrox {

    static {
        System.load("/data/local/tmp/librust_binder.so");
    }

    public static native IBinder getMediaProjectionTokenNative();

    public static void main(String[] args) {
        System.out.println("✅ MirrOx Server Started using main()");

        // Step 1: Call native function to get a raw MediaProjection token
        IBinder token = getMediaProjectionTokenNative();
        if (token == null) {
            System.err.println("❌ Failed to get MediaProjection token from JNI");
            return;
        }

        System.out.println("📣 JNI returned token: " + token);

        // Step 2: Get IMediaProjectionManager via ServiceManager
        IBinder mgrBinder = ServiceManager.getService("media_projection");
        if (mgrBinder == null) {
            System.err.println("❌ Failed to get media_projection service");
            return;
        }

        IMediaProjectionManager mgr = IMediaProjectionManager.Stub.asInterface(mgrBinder);

        try {
            // Step 3: Convert the raw Binder into an IMediaProjection
            IMediaProjection projection = IMediaProjection.Stub.asInterface(token);
            System.out.println("🎥 MediaProjection instance: " + projection);

            // TODO: Start using `projection` with MediaCodec / encoder
        } catch (Exception e) {
            e.printStackTrace();
            System.err.println("❌ Failed to create MediaProjection from token");
        }
    }
}
