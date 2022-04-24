import { Test, TestingModule } from '@nestjs/testing';
import { ControlpresetsService } from './controlpresets.service';

describe('ControlpresetsService', () => {
  let service: ControlpresetsService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ControlpresetsService],
    }).compile();

    service = module.get<ControlpresetsService>(ControlpresetsService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });
});
