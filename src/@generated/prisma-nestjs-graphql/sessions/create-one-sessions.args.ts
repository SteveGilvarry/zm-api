import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SessionsCreateInput } from './sessions-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneSessionsArgs {

    @Field(() => SessionsCreateInput, {nullable:false})
    @Type(() => SessionsCreateInput)
    data!: SessionsCreateInput;
}
