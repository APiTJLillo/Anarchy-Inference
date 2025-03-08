// Create UI library
λ ui {
    // Create window
    ƒ □(title, width, height) {
        ⬢.□(title, width, height);
    };

    // Add text
    ƒ ⬚(text) {
        ⬢.⬚(text);
    };

    ƒ start() {
        ⬢.□("Anarchy UI", 800, 600);
        ⬢.⬚("Hello World");
    };
}

// Start the app
ui.start();