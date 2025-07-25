#include <cstdarg>
#include <fstream>
#include <iostream>
#include "mystl.hpp"
#include "os.hpp"

using std::cout;
using std::endl;

using str_arg = const std::string&;

std::string get_output_name(const std::string& input_file);
std::string format_str(const std::string text, ...);
std::string get_transcription(const std::string& output);
std::string read_file(const std::string& filename);
std::string transcription_to_srt(const std::string& transcript);
void write_file(const std::string& filename, const std::string& text);

int main(int argc, char** argv)
{
	if (argc > 1) {
		std::string filename = argv[1];
		std::string output = get_output_name(filename) + ".wav";
		std::string hw_encoder;
		std::string encoding_flag;
	#if defined(OS_MACOS)
		hw_encoder = "auto";
		encoding_flag = " -c:v h264_videotoolbox ";
	#elif defined(OS_LINUX)
		hw_encoder = "vaapi";
		encoding_flag = " -c:v h264_vaapi -vf 'format= nv12, hwupload' ";
	#elif defined(OS_WINDOWS)
		hw_encoder = "qsv";
		encoding_flag = " -c:v h264_qsv ";
	#else
		hw_encoder = "auto";
		encoding_flag = " ";
	#endif
		std::string extraction_cmd = get_extraction_cmd(hw_encoder, filename, encoding_flag, output);
		std::pair<int, std::string> res = OS::run_command(extraction_cmd);
		if (res.first != 0) log_err_and_exit("There was a problem extracting the transcript from the video.", res.second);
		// The whisper command is: <path>/<to>/whisper-cpp/build/bin/whisper-cli -m <path>/<to>/whisper-cpp/models/ggml-base.en.bin -f <WAV_file>
		std::string whisper_cmd = read_file("whisper-cmd.txt") + output;
		std::pair<int, std::string> res2 = OS::run_command(whisper_cmd);
		if (res2.first != 0) log_err_and_exit("There was a problem running Whisper.", res2.second);
		cout << "Getting transcription..." << endl;
		std::string transcription = get_transcription(res2.second);
		cout << "Transcription:\n" << transcription << "\n" << endl;
		cout << "Converting to SRT format..." << endl;
		transcription = transcription_to_srt(transcription);
		cout << "Transcription converted successfully!\n" << endl;
		cout << "Writing SRT file..." << endl;
		write_file("transcript.srt", transcription);
		cout << "File written successfully!\n" << endl;
		// Absolute path where `transcript.srt` can be found.
		std::string transcript_path = read_file("transcript-path.txt");
		std::string caption_cmd = get_caption_cmd(argc, argv, filename, transcript_path);
		cout << "Embedding captions..." << endl;
		std::pair<int, std::string> res3 = OS::run_command(caption_cmd);
		if (res3.first == 0) cout << "Captions added successfully!\n" << endl;
		std::string remove_cmd = "rm " + output;
		std::pair<int, std::string> res4 = OS::run_command(remove_cmd);
	}
	else {
		cout << "Usage: blcap <video_file>" << endl;
	}
	return 0;
}

std::string format_str(const std::string text,  ...)
{
	va_list args;
	va_start(args, text);
	int size = vsnprintf(NULL, 0, text.c_str(), args);
	// Get required size
	va_end(args);
	// `args` is consumed here
	if (size < 0) return "";
	std::string buffer(size, '\0');
	// Allocate std::string with correct size
	va_list args2;
	va_start(args2, text);
	vsnprintf(&buffer[0], size + 1, text.c_str(), args2);
	// Fill buffer
	va_end(args2);
	return buffer;
}

std::string get_output_name(const std::string& input_file)
{
	size_t pos = input_file.find(".");
	if (pos == -1) return input_file;
	return input_file.substr(0, pos);
}

int transcribe_audio(const std::string& audioFile, const std::string& transcriptFile)
{
	std::string command = "./whisper -m models/medium.en.bin -f " + audioFile + " -otxt";
	return system(command.c_str());
}

std::string read_file(const std::string& filename)
{
	std::ifstream file(filename);
	// Open in read mode
	if (!file) {
		std::cerr << "Error opening file: " << filename << std::endl;
		exit(1);
	}
	return std::string((std::istreambuf_iterator<char>(file)), std::istreambuf_iterator<char>());
}

void write_file(const std::string& filename, const std::string& text)
{
	std::ofstream file(filename.c_str());
	if (file.is_open()) {
		file << text;
		return;
	}
	std::cerr << "Error opening file" << endl;
	exit(1);
}

std::string get_transcription(const std::string& output)
{
	my::string temp = output;
	my::vector<my::string> lines = temp.split("\n");
	my::vector<my::string> temp_vec;
	for (const my::string& line : lines) {
		if (line.str().starts_with("[")) temp_vec.emplace(line);
	}
	return temp_vec.join("\n");
}

my::string convert_line(const my::string& line, size_t current)
{
	size_t end_bracket = line.str().find("]") + 1;
	my::string timestamp = line.str().substr(0, end_bracket);
	while (timestamp.contains(".")) {
		size_t pos = timestamp.str().find(".");
		timestamp[pos] = ', ';
	}
	timestamp = timestamp.str().substr(1, timestamp.size() - 2);
	my::string text = line.str().substr(end_bracket + 1);
	text = text.trim();
	my::string result = format_str("%d\n%s\n%s", current, timestamp.str().c_str(), text.str().c_str());
	return result;
}

std::string transcription_to_srt(const std::string& transcript)
{
	my::string temp = transcript;
	my::vector<my::string> lines = temp.split("\n");
	size_t current = 1;
	my::vector<my::string> temp_vec;
	for (my::string line : lines) {
		my::string new_line = convert_line(line, current++);
		temp_vec.emplace(new_line);
	}
	return temp_vec.join("\n\n").trim();
}

enum class Flag 
{
	BURN,
};

std::string get_caption_cmd(int argc, char** argv, str_arg filename, str_arg transcript_path)
{
	std::string result;
	if (argc > 2) {
		Flag f;
		for (size_t i = 0; i < argc; i++) {
			if (argv[i][0] == '-') {
				std::string flag = get_flag(argv[i]);
				if (flag == "burn") {
					f = Flag::BURN;
					break;
				}
			}
		}
		switch (f) {
		case Flag::BURN: 
			result = construct_burn_cmd(filename, transcript_path);
			break;
		default: 
			cout << "Invlaid flag." << endl;
			break;
		}
	}
	else {
		result = format_str("ffmpeg -y -i %s -i %s -c:v copy -c:a copy -c:s mov_text %s_w_captions.mp4", filename.c_str(), transcript_path.c_str(), get_output_name(filename).c_str());
	}
	return result;
}

std::string construct_burn_cmd(str_arg hw_encoder, str_arg filename, str_arg encoding_flag, str_arg transcript_path)
{
	std::string result;
	result = format_str("ffmpeg -y -hwaccel %s -i %s%s-vf subtitles= %s -c:a copy %s_w_captions.mp4", hw_encoder.c_str(), filename.c_str(), encoding_flag.c_str(), transcript_path.c_str(), get_output_name(filename).c_str());
	return result;
}

std::string get_flag(const char* arg)
{
	std::string result = "";
	size_t len = strlen(arg);
	for (size_t i = 0; i < len; i++) {
		if (arg[i] == '-') continue;
		result = arg[i];
	}
	return result;
}

std::string get_extraction_cmd(str_arg hw_encoder, str_arg filename, str_arg encoding_flag, str_arg output)
{
	std::string result = format_str("ffmpeg -y -hwaccel %s -i %s%s-vn -acodec pcm_s16le -ar 16000 -ac 1 %s", hw_encoder.c_str(), filename.c_str(), encoding_flag.c_str(), output.c_str());
	return result;
}

void log_err_and_exit(str_arg text, str_arg err_message)
{
	std::cerr << text << endl;
	std::cerr << err_message << endl;
	exit(1);
}