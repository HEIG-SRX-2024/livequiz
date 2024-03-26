import { AfterViewInit, Component, ViewChild } from '@angular/core';
import { ConnectionService, Result, ResultState } from '../services/connection.service';
import { MatListModule, MatSelectionList } from '@angular/material/list';
import { MatGridListModule } from '@angular/material/grid-list';
import { CommonModule } from '@angular/common';
import { MatTableModule } from '@angular/material/table';
import { MatSlideToggle, MatSlideToggleChange, MatSlideToggleModule } from '@angular/material/slide-toggle';
import { animals, colors, uniqueNamesGenerator } from 'unique-names-generator';
import { Buffer } from 'buffer';
import { Questionnaire, QuestionnaireService } from '../services/questionnaire.service';
import { UserService } from '../services/user.service';
import { RouterLink } from '@angular/router';
import { environment } from '../../environments/environment';

@Component({
  selector: 'app-admin',
  standalone: true,
  imports: [MatListModule, MatSelectionList, MatGridListModule, CommonModule, MatTableModule,
    MatSlideToggleModule, RouterLink],
  templateUrl: './admin.component.html',
  styleUrl: './admin.component.scss'
})
export class AdminComponent implements AfterViewInit {
  @ViewChild('toggleEditAllowed') toggleEditAllowed!: MatSlideToggle;
  @ViewChild('toggleShowResults') toggleShowResults!: MatSlideToggle;
  users: Result[] = [];
  displayedColumns: number[] = [];
  showResults = false;
  editAllowed = true;
  selectedClasses: string[][] = [];
  questionnaire = new Questionnaire("");
  title = "Not available";
  addTestUsers = environment.addTestUsers;

  constructor(private connection: ConnectionService, private qservice: QuestionnaireService,
    private user: UserService) {
  }
  async ngAfterViewInit() {
    this.qservice.loaded.subscribe((q) => {
      this.questionnaire = q;
      this.update();
    });
    this.connection.showResults.subscribe((show) => {
      this.showResults = show;
      this.toggleShowResults.setDisabledState(false);
      this.updateSelectedClass();
    });
    this.connection.editAllowed.subscribe((edit) => {
      this.editAllowed = edit;
      this.toggleEditAllowed!.setDisabledState(false);
    });
    this.connection.answersHash.subscribe(() => {
      this.update();
    })
  }

  async update() {
    this.users = await this.connection.getResults();
    this.users.forEach((u) => {
      for (; u.answers.length < this.questionnaire.questions.length; u.answers.push("empty")) { }
    });
    this.users.sort((a, b) => a.name.localeCompare(b.name));
    this.updateSelectedClass();
    this.title = this.qservice.loaded.value.chapter;
    this.displayedColumns = this.questionnaire.questions.map((_, i) => i);
  }

  async updateQuestionnaire() {
    await this.connection.updateQuestionnaire(this.user.secret);
  }

  async updateSelectedClass() {
    this.selectedClasses = this.users.map((user) =>
      user.answers ? user.answers.map((c) => {
        let cl = c;
        if (c === "correct" && !this.showResults) {
          cl = "answered";
        }
        return `userAnswer_${cl}`;
      }) : [""]
    )
  }

  async addUser() {
    const secret = Buffer.alloc(32);
    self.crypto.getRandomValues(secret);

    const name = uniqueNamesGenerator({
      dictionaries: [colors, animals],
      separator: '-',
    });
    await this.connection.updateName(secret, name);
    for (let q = 0; q < this.questionnaire.questions.length; q++) {
      const result: ResultState = ["empty", "answered", "correct"][Math.floor(Math.random() * 3)] as ResultState;
      const question = this.questionnaire.questions[q];
      question.shuffle();
      const choicesNbr = Math.floor(Math.random() * (question.maxChoices + 1));
      const choices = question.original.slice(0, choicesNbr);
      await this.connection.updateQuestion(secret, q, result, choices);
    }
    this.update();
  }

  editAllowedUpdate(event: MatSlideToggleChange) {
    this.connection.setEditAllowed(this.user.secret, event.checked);
    event.source.checked = this.connection.editAllowed.value;
    event.source.disabled = true;
  }

  showResultsUpdate(event: MatSlideToggleChange) {
    this.connection.setShowAnswers(this.user.secret, event.checked);
    event.source.checked = this.connection.showResults.value;
    event.source.disabled = true;
    this.updateSelectedClass();
  }
}
