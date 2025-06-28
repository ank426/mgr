all: bin/mgr

bin/mgr: bin force_recompile
	gcc -o bin/mgr src/*.c -lm -lzip -lSDL3 -lSDL3_image -lSDL3_ttf

bin:
	mkdir bin

clean:
	rm -rf bin

run:
	bin/mgr

force_recompile:
	@echo "Recompiling..."
