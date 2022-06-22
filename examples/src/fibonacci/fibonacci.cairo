func main():
    [ap] = 50; ap++
    call fib_wrapper
    ret
end

func fib_wrapper(n):
    # Call fib(1, 1, 1000).
    [ap] = 1; ap++
    [ap] = 1; ap++
    [ap] = 1000; ap++
    call fib

    # Make sure the 1000th Fibonacci number is 222450955505511890955301767713383614666194461405743219770606958667979327682.
    [ap - 1] = 222450955505511890955301767713383614666194461405743219770606958667979327682 
    if n != 0:
        [ap] = n - 1; ap++
        call fib_wrapper
    end
    ret
end

func fib(first_element, second_element, n) -> (res : felt):
    jmp fib_body if n != 0
    [ap] = second_element; ap++
    ret

    fib_body:
    [ap] = second_element; ap++
    [ap] = first_element + second_element; ap++
    [ap] = n - 1; ap++
    call fib
    ret
end
