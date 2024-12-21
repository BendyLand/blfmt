#include "gui.hpp"

void initWindowSettings(QWidget& window)
{
	window.resize(525, 575);
	window.setWindowTitle("Installer");
	window.setStyleSheet("background-color: lightgrey;");
	// Main layout for the window
	QVBoxLayout* mainLayout = new QVBoxLayout(&window);
	// Title label
	QLabel* titleLabel = new QLabel("Installing: My Application", &window);
	titleLabel->setAlignment(Qt::AlignCenter);
	titleLabel->setStyleSheet("font-size: 20px; font-weight: bold;");
	mainLayout->addWidget(titleLabel);
	// Create stacked widget for pages
	QStackedWidget* stackedWidget = new QStackedWidget(&window);
	mainLayout->addWidget(stackedWidget);
	// Page 1
	QWidget* page1 = new QWidget(stackedWidget);
	// Set parent to stackedWidget
	QVBoxLayout* page1Layout = new QVBoxLayout(page1);
	page1Layout->addWidget(new QLabel("Welcome to the installer!", page1));
	QPushButton* nextButton1 = new QPushButton("Next", page1);
	page1Layout->addWidget(nextButton1);
	stackedWidget->addWidget(page1);
	// Page 2
	QWidget* page2 = new QWidget(stackedWidget);
	// Set parent to stackedWidget
	QVBoxLayout* page2Layout = new QVBoxLayout(page2);
	page2Layout->addWidget(new QLabel("Installation in progress...", page2));
	QPushButton* nextButton2 = new QPushButton("Next", page2);
	page2Layout->addWidget(nextButton2);
	stackedWidget->addWidget(page2);
	// Page 3
	QWidget* page3 = new QWidget(stackedWidget);
	// Set parent to stackedWidget
	QVBoxLayout* page3Layout = new QVBoxLayout(page3);
	page3Layout->addWidget(new QLabel("Installation complete!", page3));
	QPushButton* finishButton = new QPushButton("Finish", page3);
	page3Layout->addWidget(finishButton);
	stackedWidget->addWidget(page3);
	// Connect buttons to change pages
	QObject::connect(nextButton1, &QPushButton::clicked, [=]() {
		stackedWidget->setCurrentIndex(1);
		// Go to page 2
	});
	QObject::connect(nextButton2, &QPushButton::clicked, [=]() {
		stackedWidget->setCurrentIndex(2);
		// Go to page 3
	});
	QObject::connect(finishButton, &QPushButton::clicked, &window, &QWidget::close);
	// Set initial page
	stackedWidget->setCurrentIndex(0);
}