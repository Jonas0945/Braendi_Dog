# Braendi_Dog 

## Documentation

1. Voraussetzungen:

- Rust& Cargo sind installiert
- Git ist installiert

2. Projekt starten 
- Repository Klonen
- cargo run (Hinweis, beim ersten Kompilieren kann dies einige Zeit dauern)
- Es öffnet sich dann die Gui

3. Spielmodi
	3.1 Lokal (Alle Spieler spielen ma gleichen PC)

	- Im Dropdown Menü den Spielmodus auswählen
	- Lokal Starten klicen
	- Im Bot-Setup-Fenster festlegen wer Mensch, wer Bot ist 
	- Spiel starten! klicken
	3.2 Lan (gleiches Netzwerk)

	Auf dem Host-PC:
		1. Spielmodus wählen, Namen eingeben
		2. Bind Adresse auf '0.0.0.0:833' lassen
		3. Spiel Hosten klicken
		Lokale IP herausfinden:
			- Windows: cmd -> ipconfig eingeben -> Enter -> IPv4 Adresse an Mitspieler weitergeben
			- Mac:
				- Öffne die Spotlight-Suche (Cmd + Leertaste).
				- Tippe Terminal ein und öffne die App.
				- Wenn du über WLAN verbunden bist, tippe: ipconfig getifaddr en0
				- Wenn du über Kabel (Ethernet) verbunden bist, tippe: ipconfig getifaddr en1 (oder en2).
			- Linux: Terminal öffnen, 'hostname -I'
		Die IP mit dem Zusatz :8333 (Oder falls verwendet einen anderen Port) auf den client PCs im Join Feld eingeben 
		Spiel Beitreten klicken
	3.3 Internet – Port-Forwarding

	- Router-Oberfläche öffnen (meist 192.168.1.1)
	- Port-Forwarding Regel: TCP, externer Port 8333, interner Port 8333, interne IP = LAN-IP des Host-PCs
	- Öffentliche IP auf whatismyip.com nachschauen
	- Diese IP + Port (z.B. 85.123.45.67:8333) an Mitspieler weitergeben
	3.4 Internet – bore

	- cargo install bore-cli einmalig installieren
	- Vor dem Spielen in separatem Terminal: bore local 8333 --to bore.pub
	- bore zeigt z.B. listening at bore.pub:54321 – diese Adresse aus dem Terminal kopieren und an Mitspieler weitergeben
	- Im Spiel normal "Spiel Hosten" klicken
	- Mitspieler tragen bore.pub:54321 ins Join-Feld ein
	- bore-Fenster offen lassen solange gespielt wird – Port ändert sich bei jedem Neustart


	4.2 Bots

	- Zufalls-Bot: Wählt zufällig einen gültigen Zug aus allen möglichen
	- Eval-Bot: Bewertet alle möglichen Züge mit einer Funktion und wählt den besten


	5. Bedienung

	- Karte unten anklicken → hebt sich hervor
	- Aktion rechts wählen (Move, Place, Interchange, Remove, Trade...)
	- Bei Move: Startfeld mit Figue anklicken → Markierungen erscheinen → Zielfeld anklicken
	- Bei Place: einfach "Legen" klicken, keine Feldauswahl nötig
	- Bei Interchange: zwei Felder nacheinander anklicken
	- Karte abwerfen: "Abwerfen (Remove)" wenn kein Zug möglich
	- Aktion abbrechen: "Abbrechen" Button rechts


	6. Troubleshooting

	- Verbindung fehlgeschlagen → IP/Port prüfen, Firewall auf Host-PC prüfen
	- Warte auf Brettdaten bleibt stehen → zurück zum Menü, neu verbinden
	- Kein Ton → lobby.mp3/win.mp3 fehlen, Spiel läuft trotzdem
	- bore-Port ändert sich → bore neu starten, neue Nummer weitergeben
	- cargo run langsam → cargo run --release verwenden
