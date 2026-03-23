Feature: Test de l'application Electron

  Scenario: Vérifier la page d'accueil
    Given the window size is 1920x1080

    When I navigate to "https://example.com"
    And I take a screenshot "01-accueil.png"
    And I wait 1 seconds

    Then the page should contain "Example Domain"
