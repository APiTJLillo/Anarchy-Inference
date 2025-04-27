
// Error handling test
m{
  main() {
    try {
      result = 10 / 0;
      print("This should not print");
    } catch(e) {
      print("Caught division by zero error");
    }
    return 0;
  }
}
