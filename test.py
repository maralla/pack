import sys
import time

def spinning_cursor():
    while True:
        for cursor in '|/-\\':
            yield cursor

spinner = spinning_cursor()
for _ in range(50):
    sys.stdout.write(spinner.next())
    sys.stdout.flush()
    time.sleep(0.1)
    sys.stdout.write('\b')
