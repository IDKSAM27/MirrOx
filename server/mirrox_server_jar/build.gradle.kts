val androidHome: String? = System.getenv("ANDROID_HOME")
val androidJar = file("C:/Users/Sampreet/AppData/Local/Android/Sdk/platforms/android-35/android.jar")

dependencies {
    compileOnly(files(androidJar))
}

plugins {
    java
}

group = "com.mirrox"
version = "1.0"

repositories {
    mavenCentral()
}

java {
    sourceCompatibility = JavaVersion.VERSION_1_8
    targetCompatibility = JavaVersion.VERSION_1_8
}

tasks.register<Jar>("buildMirroxJar") {
    archiveBaseName.set("mirrox_server")
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE

    from(sourceSets["main"].output)

    manifest {
        attributes["Main-Class"] = "com.mirrox.server.MirroxServer"
    }
}
