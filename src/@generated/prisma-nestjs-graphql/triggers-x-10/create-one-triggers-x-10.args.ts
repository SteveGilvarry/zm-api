import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { TriggersX10CreateInput } from './triggers-x-10-create.input';

@ArgsType()
export class CreateOneTriggersX10Args {

    @Field(() => TriggersX10CreateInput, {nullable:false})
    data!: TriggersX10CreateInput;
}
