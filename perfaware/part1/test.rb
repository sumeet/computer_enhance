#!/usr/bin/ruby
require "tempfile"

DECODER_LISTINGS = [
  "listing_0037_single_register_mov",
  "listing_0038_many_register_mov",
  "listing_0039_more_movs",
  "listing_0040_challenge_movs",
  "listing_0041_add_sub_cmp_jnz",
  # unimplemented:
  # "listing_0042_completionist_decode",
]
#DECODER_LISTINGS = []

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

puts "running #{DECODER_LISTINGS.size} decoder tests..."
DECODER_LISTINGS.each do |listing|
  Tempfile.create do |new_listing_file|
    Tempfile.create do |new_output_file|
      # decode the binary and put it in new_listing_file
      new_listing_file.write(run(listing))
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
  "listing_0044_register_movs",
  # "listing_0045_challenge_register_movs",
  "listing_0046_add_sub_cmp",
  "listing_0048_ip_register",
  "listing_0049_conditional_jumps",
  # we might as well try this one, looks not that bad:
  # "listing_0050_challenge_jumps", 
  "listing_0051_memory_mov",
  "listing_0052_memory_add_loop",
  "listing_0053_add_loop_challenge",
  "listing_0054_draw_rectangle",
]

State = Struct.new(:regs, :flags)

def ingest_state(output)
  lines = output.lines(chomp: true)

  regs = lines.filter_map do |line|
    next unless line.start_with? "      "
    reg, val, _ = line.split " "
    reg.delete_suffix! ":"
    [reg, Integer(val)]
  end.to_h

  flags = ""
  flags_line, = lines.grep(/^   flags:/)
  if !flags_line.nil?
    flags = flags_line.delete_prefix "   flags: "
    flags = flags.chars.sort.join("")
  end

  State.new(regs, flags)
end

def compare(want_state, got_state)
  is_regs_match = want_state.regs.all? do |reg, value|
    got_state.regs[reg] == value
  end
  is_flags_match = want_state.flags == got_state.flags
  is_regs_match && is_flags_match
end

puts
puts "running #{SIM_LISTINGS.size} simulator tests..."
SIM_LISTINGS.each do |listing|
  reference_registers = ingest_state(File.read "#{listing}.txt")
  our_registers = ingest_state(run(listing, sim: true))
  if compare(reference_registers, our_registers)
    puts "#{listing} pass"
  else
    $failed = true
    puts "#{listing} sim fail, here's the diff of the state"
    puts "expected: #{reference_registers}"
    puts "got: #{our_registers}"
  end
end

exit 1 if $failed
