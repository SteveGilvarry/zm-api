import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsCreateManyInput } from './controls-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyControlsArgs {

    @Field(() => [ControlsCreateManyInput], {nullable:false})
    @Type(() => ControlsCreateManyInput)
    data!: Array<ControlsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
