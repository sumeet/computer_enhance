#include <array>
#include <cassert>
#include <cmath>
#include <fstream>
#include <iostream>
#include <x86intrin.h>

const std::array<char, 2> X0 = {'x', '0'};
const std::array<char, 2> X1 = {'x', '1'};
const std::array<char, 2> Y0 = {'y', '0'};
const std::array<char, 2> Y1 = {'y', '1'};

typedef double f64;
typedef unsigned long long u64;

const f64 EARTH_RADIUS = 6372.8;

#include "listing_0065_haversine_formula.cpp"
#include "listing_0070_platform_metrics.cpp"

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

u64 ApproxCPUTimerFreq() {
  u64 MillisecondsToWait = 100;

  u64 OSFreq = GetOSTimerFreq();

  u64 CPUStart = ReadCPUTimer();
  u64 OSStart = ReadOSTimer();
  u64 OSEnd = 0;
  u64 OSElapsed = 0;
  u64 OSWaitTime = OSFreq * MillisecondsToWait / 1000;
  while (OSElapsed < OSWaitTime) {
    OSEnd = ReadOSTimer();
    OSElapsed = OSEnd - OSStart;
  }

  u64 CPUEnd = ReadCPUTimer();
  u64 CPUElapsed = CPUEnd - CPUStart;
  u64 CPUFreq = 0;
  if (OSElapsed) {
    CPUFreq = OSFreq * CPUElapsed / OSElapsed;
  }
  return CPUFreq;
}

u64 rtdsc() { return __rdtsc(); }

int main(int argc, char **argv) {
  auto cpu_timer_freq = ApproxCPUTimerFreq();

  auto start_time = rtdsc();

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

  auto end_time = rtdsc();
  auto elapsed_time = end_time - start_time;
  std::cout << "Elapsed time: " << (f64)elapsed_time / (f64)cpu_timer_freq
            << " seconds (CPU Timer Freq: " << cpu_timer_freq << ")"
            << std::endl;

  return 0;
}