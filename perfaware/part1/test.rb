#!/usr/bin/ruby
require "tempfile"

LISTINGS = [
  "listing_0037_single_register_mov",
  "listing_0038_many_register_mov",
  "listing_0039_more_movs",
  "listing_0040_challenge_movs",
  "listing_0041_add_sub_cmp_jnz",
]

def readbin path
  File.read path, encoding: "BINARY"
end

#### BEGINNING OF SCRIPT

# compile the program first
system("cd sim && cargo build")

$failed = false

LISTINGS.each do |listing|
  Tempfile.create do |new_listing_file|
    Tempfile.create do |new_output_file|
      # decode the binary and put it in new_listing_file
      new_listing_file.write(`./sim/target/debug/sim #{listing}`)
      new_listing_file.close

      `nasm #{new_listing_file.path} -o #{new_output_file.path}`
      if readbin(new_output_file.path) == readbin(listing)
        puts "#{listing} pass"
      else
        $failed = true
        puts "#{listing} fail, here's the diff of the assemblies"
        print `diff -U10 <(cat #{listing}.asm | grep -v '^$' | grep -v '^;') <(cat #{new_listing_file.path})`
      end
    end
  end
end

exit 1 if $failed
