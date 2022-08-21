import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { TriggersX10CreateManyInput } from './triggers-x-10-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyTriggersX10Args {

    @Field(() => [TriggersX10CreateManyInput], {nullable:false})
    @Type(() => TriggersX10CreateManyInput)
    data!: Array<TriggersX10CreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
