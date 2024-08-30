package io.github.alphagodzilla.partitionlink;

import java.io.File;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.StandardCopyOption;
import java.util.concurrent.atomic.AtomicBoolean;

public class LibLoader {
    private static final String LIBNAME = "libpartition_link_java";

    private static final AtomicBoolean LOADED = new AtomicBoolean(false);

    private LibLoader() {
    }

    public static void load() {
        if (LOADED.get()) {
            return;
        }
        synchronized (LibLoader.class) {
            // double check
            if (LOADED.get()) {
                return;
            }
            try {
                // trye load
                tryloadFromLibPath();
                tryLoadFromResourceDir();
            } catch (Exception exception) {
                throw new RuntimeException(exception);
            }
            LOADED.set(true);
        }
    }

    private static void tryloadFromLibPath() {
        // -Djava.library.path
        System.loadLibrary(LIBNAME);
    }

    private static void tryLoadFromResourceDir() {
        // from resouce dir
        String libName = LIBNAME + libPlatformSuffix();
        InputStream stream = LibLoader.class.getClassLoader().getResourceAsStream(libName);
        if (stream == null) {
            return;
        }
        try {
            final File tmpFile = File.createTempFile(libName.split(".")[0], libPlatformSuffix());
            tmpFile.deleteOnExit();
            Files.copy(stream, tmpFile.toPath(), StandardCopyOption.REPLACE_EXISTING);
            System.load(tmpFile.getAbsolutePath());
        } catch (Exception exception) {
            throw new RuntimeException(exception);
        }

    }

    private static String libPlatformSuffix() {
        final String os = System.getProperty("os.name").toLowerCase();
        if (os.startsWith("mac")) {
            return ".dylib";
        } else if (os.startsWith("windows")) {
            return ".dll";
        } else {
            return ".so";
        }
    }
}
