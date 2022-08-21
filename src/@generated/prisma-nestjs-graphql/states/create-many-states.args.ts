import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesCreateManyInput } from './states-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyStatesArgs {

    @Field(() => [StatesCreateManyInput], {nullable:false})
    @Type(() => StatesCreateManyInput)
    data!: Array<StatesCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
