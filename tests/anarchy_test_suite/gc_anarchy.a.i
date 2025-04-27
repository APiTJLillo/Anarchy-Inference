
// Garbage collection test
m{
  main() {
    // Create objects that should be garbage collected
    for(i = 0; i < 100; i = i + 1) {
      temp_obj = {"id": i, "data": "This is temporary data"};
    }
    print("Created 100 temporary objects that should be garbage collected");
    print("Memory should be reclaimed after objects go out of scope");
    return 0;
  }
}
