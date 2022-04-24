import { Test, TestingModule } from '@nestjs/testing';
import { EventsummariesService } from './eventsummaries.service';

describe('EventsummariesService', () => {
  let service: EventsummariesService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [EventsummariesService],
    }).compile();

    service = module.get<EventsummariesService>(EventsummariesService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });
});
