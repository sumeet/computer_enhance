all: process generate

generate: generate.cpp
	g++ -O3 -o generate generate.cpp

process: process.cpp
	g++ -O3 -o process process.cpp

simple:
	g++ -O3 -o simple listing_0067_simple_haversine.cpp

clean:
	rm -f process generate
