int fib(int n) {
    if (n <= 2) {
        return n - 1;
    } else {
        return fib(n - 1) + fib(n - 2);
    }
}

int main() {
    return fib(10);  // 34
}
