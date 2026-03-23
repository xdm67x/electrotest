Feature: Test de l'application Electron

  Scenario: Vérifier la page d'accueil
    Given the window size is 800x600

    When I navigate to "https://google.fr"
    And I wait 1 seconds
    And I take a screenshot "01-google.png"
    And I click on "À propos"
    And I wait 1 seconds
    And I take a screenshot "02-google.png"

    Then the page should contain "Google Maps just got a major Gemini upgrade"
