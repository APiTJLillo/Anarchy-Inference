// Helper function to test the UI integration
function testUIIntegration() {
    console.log("Starting UI integration test");
    // Our test code from test_input.txt
    const testCode = `λui{
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
}`;

    // Find the code editor textarea
    const editor = document.querySelector('.code-editor');
    if (editor) {
        editor.value = testCode;
        // Trigger change event
        editor.value = testCode;
        const event = new Event('change', { bubbles: true });
        editor.dispatchEvent(event);
        console.log('Test code set in editor');

        // Monitor window creation
        const observer = new MutationObserver((mutations) => {
            mutations.forEach((mutation) => {
                if (mutation.type === 'childList') {
                    mutation.addedNodes.forEach((node) => {
                        if (node.classList && node.classList.contains('anarchy-window')) {
                            console.log('New window created:', node);
                        }
                    });
                }
            });
        });

        observer.observe(document.body, { childList: true, subtree: true });
    }

    // Wait a moment then find and click the execute button
    const executeBtn = document.querySelector('.anarchy-button');
    if (executeBtn) {
        setTimeout(() => {
            console.log('Test sequence starting...');
            executeBtn.click();
            console.log('Execute button clicked, waiting for response...');
            
            // Poll for window creation
            let attempts = 0;
            const maxAttempts = 30; // Increased timeout for slower systems
            const checkForWindow = setInterval(() => {
                const uiWindow = document.querySelector('.anarchy-window[title="Test Window"]');
                if (uiWindow) {
                    console.log('UI window found:', uiWindow.getAttribute('title'));
                    console.log('Testing window elements...');
                    // Verify window content
                    const windowContent = uiWindow.querySelector('.anarchy-window-content');
                    const textElements = windowContent.querySelectorAll('.anarchy-text');
                    const inputElement = windowContent.querySelector('.anarchy-input');
                    
                    console.log('Found window with title:', uiWindow.getAttribute('title'));
                    console.log('Text elements:', Array.from(textElements).map(el => el.textContent));
                    console.log('Input element:', inputElement ? 'found' : 'not found');
                    console.log('Input placeholder:', inputElement?.getAttribute('placeholder'));
                    
                    // Verify specific content
                    const expectedTexts = [
                        'Hello from Anarchy!',
                        'This is a text component'
                    ];
                    
                    const allTextFound = expectedTexts.every(text => 
                        Array.from(textElements).some(el => el.textContent === text)
                    );
                    
                    const inputPlaceholder = inputElement?.getAttribute('placeholder');
                    const hasCorrectInput = inputPlaceholder === 'Enter something...';
                    
                    if (allTextFound && hasCorrectInput) {
                        console.log('✅ UI integration test passed: All content verified');
                    } else {
                        console.log('❌ UI integration test failed: Content mismatch');
                    }
                    clearInterval(checkForWindow);
                } else if (++attempts >= maxAttempts) {
                    console.log('Timeout waiting for UI window');
                    clearInterval(checkForWindow);
                }
            }, 250); // Decreased interval for more frequent checks
        }, 500);
    }
}

window.testUIIntegration = testUIIntegration;
