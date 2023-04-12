#!/usr/bin/ruby
require "tempfile"

DECODER_LISTINGS = [
  "listing_0037_single_register_mov",
  "listing_0038_many_register_mov",
  "listing_0039_more_movs",
  "listing_0040_challenge_movs",
  "listing_0041_add_sub_cmp_jnz",
  # unimplemented:
  # "listing_0043_immediate_movs.txt",
]
DECODER_LISTINGS = []

def readbin path
  File.read path, encoding: "BINARY"
end

#### BEGINNING OF SCRIPT

# compile the program first
system("cd sim && cargo build")

def run(filename, sim: false)
  if sim
    `./sim/target/debug/sim #{filename} -exec`
  else
    `./sim/target/debug/sim #{filename}`
  end
end

$failed = false

#### DECODER ONLY TESTS ################
DECODER_LISTINGS.each do |listing|
  Tempfile.create do |new_listing_file|
    Tempfile.create do |new_output_file|
      # decode the binary and put it in new_listing_file
      new_listing_file.write(run_sim(listing))
      new_listing_file.close

      `nasm #{new_listing_file.path} -o #{new_output_file.path}`
      if readbin(new_output_file.path) == readbin(listing)
        puts "#{listing} pass"
      else
        $failed = true
        puts "#{listing} decoder fail, here's the diff of the assemblies"
        print `diff -U10 <(cat #{listing}.asm | grep -v '^$' | grep -v '^;') <(cat #{new_listing_file.path})`
      end
    end
  end
end

exit 1 if $failed

SIM_LISTINGS = [
  "listing_0043_immediate_movs",
]

def ingest_registers(output)
  output.lines(chomp: true).filter_map do |line|
    next unless line.start_with? "      "
    reg, val, _ = line.split " "
    [reg.delete_suffix(":"), Integer(val)]
  end.to_h
end

SIM_LISTINGS.each do |listing|
  reference_registers = ingest_registers(File.read "#{listing}.txt")
  our_registers = ingest_registers(run(listing, sim: true))
  if reference_registers != our_registers
    $failed = true
    puts "#{listing} sim fail, here's the diff of the state"
    puts "expected: #{reference_registers}"
    puts "got: #{our_registers}"
  end
end

exit 1 if $failed
