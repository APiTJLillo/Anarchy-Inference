λui{
    ƒtest(){
        □("Test Window", 400, 300);
        ⬚("Hello from Anarchy!");
        ✎("This is a text component");
        ⌨("Enter something...", (input) => {
            ✎("You typed: " + input);
        });
    }

    ƒstart(){
        test();
        ⟼(⊤);
    }
}

λmain{
    ƒmain(){
        ιui = ui.start();
        ⟼(ui);
    }
}
