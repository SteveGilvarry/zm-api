import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { TriggersX10WhereInput } from './triggers-x-10-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyTriggersX10Args {

    @Field(() => TriggersX10WhereInput, {nullable:true})
    @Type(() => TriggersX10WhereInput)
    where?: TriggersX10WhereInput;
}
