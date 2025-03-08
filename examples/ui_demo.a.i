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

    // Add button
    ƒ ⚈(text, handler) {
        ⬢.⚈(text, handler);
    };

    // Add input
    ƒ ⌸(placeholder, handler) {
        ⬢.⌸(placeholder, handler);
    };

    ƒ start() {
        □("Anarchy UI", 800, 600);
        ⬚("Hello World");
        
        ι count = 0;
        
        ⚈("Click me", λ _ {
            count = count + 1;
            ⬚("Clicked " + count + " times");
        });
        
        ⌸("Enter text", λ value {
            ⬚(value);
        });
    };
};

// Start the app
ui.start();