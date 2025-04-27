
// Function test
m{
  main() {
    print("Calling add function: " + add(3, 4));
    print("Calling greet function: " + greet("Anarchy"));
    return 0;
  }
  
  add(a, b) {
    return a + b;
  }
  
  greet(name) {
    return "Hello, " + name + "!";
  }
}
