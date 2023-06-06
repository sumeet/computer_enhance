#include <array>
#include <cassert>
#include <cmath>
#include <fstream>
#include <iostream>

const std::array<char, 2> X0 = {'x', '0'};
const std::array<char, 2> X1 = {'x', '1'};
const std::array<char, 2> Y0 = {'y', '0'};
const std::array<char, 2> Y1 = {'y', '1'};

typedef double f64;

const f64 EARTH_RADIUS = 6372.8;

#include "listing_0065_haversine_formula.cpp"

template <std::size_t N> std::array<char, N> consume_n(std::ifstream &file) {
  std::array<char, N> buf;
  file.read(buf.data(), N);
  return buf;
}

char consume_float_and_next(std::ifstream &file, f64 &out) {
  char c;
  while (file.get(c) && isspace(c))
    ;
  file.putback(c);
  file >> out;
  while (file.get(c) && isspace(c))
    ;
  return c;
}

char get_next_non_ws(std::ifstream &file) {
  char c;
  while (file.get(c) && isspace(c))
    ;
  return c;
}

void consume_literal(std::ifstream &file, const std::string &literal) {
  char c;
  // first consume leading spaces
  while (file.get(c) && isspace(c))
    ;
  for (int i = 0; i < literal.size(); i++) {
    if (c != literal[i]) {
      throw std::runtime_error("Expected " + literal + " but got " + c);
    }
    if (i < literal.size() - 1) {
      file.get(c);
    }
  }
}

int main(int argc, char **argv) {
  if (argc < 2) {
    std::cerr << "Usage: " << argv[0] << " <filename.json>" << std::endl;
    return 1;
  }

  std::ifstream file(argv[1]);
  if (!file.is_open()) {
    std::cerr << "Could not open file: " << argv[1] << std::endl;
    return 1;
  }

  consume_literal(file, "{");
  consume_literal(file, "\"pairs\"");
  consume_literal(file, ":");
  consume_literal(file, "[");

  size_t num_pairs = 0;
  f64 sum = 0.f;

  while (true) {
    f64 x0, y0, x1, y1;

    auto nextchar = get_next_non_ws(file);
    if (nextchar == ']') {
      break;
    }
    assert(nextchar == '{');
    consume_literal(file, "\"");

    while (true) {
      auto key = consume_n<2>(file);
      f64 *this_key;
      if (key == X0) {
        this_key = &x0;
      } else if (key == X1) {
        this_key = &x1;
      } else if (key == Y0) {
        this_key = &y0;
      } else if (key == Y1) {
        this_key = &y1;
      } else {
        throw std::runtime_error("Unexpected key: " +
                                 std::string(key.data(), 2));
      }
      consume_literal(file, "\"");
      consume_literal(file, ":");
      nextchar = consume_float_and_next(file, *this_key);
      if (nextchar == '}') {
        break;
      } else if (nextchar != ',') {
        throw std::runtime_error(&"Unexpected character: "[nextchar]);
      }
      consume_literal(file, "\"");
    }

    num_pairs++;
    sum += ReferenceHaversine(x0, y0, x1, y1, EARTH_RADIUS);

    nextchar = get_next_non_ws(file);
    if (nextchar == ']') {
      break;
    } else if (nextchar != ',') {
      throw std::runtime_error(&"Unexpected character: "[nextchar]);
    }
  }

  std::cout << "Average distance between pairs: " << (sum / (f64)num_pairs)
            << std::endl;
  return 0;
}