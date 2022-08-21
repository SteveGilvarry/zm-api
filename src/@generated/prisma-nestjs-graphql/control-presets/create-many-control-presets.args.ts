import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsCreateManyInput } from './control-presets-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyControlPresetsArgs {

    @Field(() => [ControlPresetsCreateManyInput], {nullable:false})
    @Type(() => ControlPresetsCreateManyInput)
    data!: Array<ControlPresetsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
