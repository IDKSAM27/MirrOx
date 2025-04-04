package com.example.android_server;

import java.io.OutputStream;
import java.net.ServerSocket;
import java.net.Socket;

public class ScreenStreamingServer {
    private static final int PORT = 8080; // Choose a port
    private ServerSocket serverSocket;
    private Socket clientSocket;
    private OutputStream outputStream;

    public void startServer() {
        new Thread(() -> {
            try {
                serverSocket = new ServerSocket(PORT);
                System.out.println("Server started, waiting for connection...");
                clientSocket = serverSocket.accept();
                outputStream = clientSocket.getOutputStream();
                System.out.println("Client connected");
            } catch (Exception e) {
                e.printStackTrace();
            }
        }).start();
    }

    public void sendFrame(byte[] data) {
        try {
            if (outputStream != null) {
                outputStream.write(data);
                outputStream.flush();
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    public void stopServer() {
        try {
            if (clientSocket != null) clientSocket.close();
            if (serverSocket != null) serverSocket.close();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}