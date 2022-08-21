import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { TriggersX10CreateInput } from './triggers-x-10-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneTriggersX10Args {

    @Field(() => TriggersX10CreateInput, {nullable:false})
    @Type(() => TriggersX10CreateInput)
    data!: TriggersX10CreateInput;
}
