all: bin/mgr

bin/mgr: bin force_recompile
	gcc -o bin/mgr src/main.c -lzip -lSDL3 -lSDL3_image -lm

bin:
	mkdir bin

clean:
	rm -rf bin

run:
	bin/mgr

force_recompile:
	@echo "Recompiling..."
