 /**
 *  Monitoring Dashboard - A simple custom monitoring dashboard.
 *  Copyright (C) 2024 Ben Landrette
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it may be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see https://www.gnu.org/licenses.
 */

#include <QApplication>
#include <QGridLayout>
#include <QLabel>
#include <QWidget>

int main(int argc, char** argv)
{
	QApplication app(argc, argv);
	QWidget window;
	window.resize(800, 600);
	window.setWindowTitle("Monitoring Dashboard");
	// Create a grid layout
	QGridLayout* layout = new QGridLayout(&window);
	// Add labels to specific grid positions
	layout->addWidget(new QLabel("CPU Monitoring", &window), 0, 0, Qt::AlignCenter);
	layout->addWidget(new QLabel("Memory Monitoring", &window), 0, 1, Qt::AlignCenter);
	layout->addWidget(new QLabel("Network Monitoring", &window), 0, 2, Qt::AlignCenter);
	layout->addWidget(new QLabel("Device", &window), 1, 1, Qt::AlignCenter);
	window.setStyleSheet("background-color: grey;");
	window.show();
	return app.exec();
}